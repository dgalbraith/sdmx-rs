<details>
<summary>XSD contract: <code>VersionableType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="VersionableType" abstract="true">
		<xs:annotation>
			<xs:documentation>VersionableType is an abstract base type for all versionable objects.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="NameableType">
				<xs:attribute name="version" type="VersionType" use="optional">
					<xs:annotation>
						<xs:documentation>This version attribute holds a version number (see common:VersionType definition for details). If not supplied, artefact is considered to be un-versioned.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute name="validFrom" type="xs:dateTime" use="optional">
					<xs:annotation>
						<xs:documentation>The validFrom attribute provides the inclusive start date for providing supplemental validity information about the version.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute name="validTo" type="xs:dateTime" use="optional">
					<xs:annotation>
						<xs:documentation>The validTo attribute provides the inclusive end date for providing supplemental validity information about the version.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
