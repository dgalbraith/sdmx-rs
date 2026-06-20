<details>
<summary>XSD contract: <code>AvailabilityConstraintType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="AvailabilityConstraintType">
		<xs:annotation>
			<xs:documentation>AvailabilityConstraintType defines the structure of an availability constraint. This type of constraint contains the actual data currently present for the referenced object and is used to express data availability either by listing the available sets of keys (DataKeySet) or a set of component values (CubeRegion), e.g., in a data source. Availability constraints should not be (semantically) versioned.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="common:AnnotableType">
				<xs:sequence>
					<xs:element name="ConstraintAttachment" type="AvailabilityConstraintAttachmentType" minOccurs="1" maxOccurs="1">
						<xs:annotation>
							<xs:documentation>ConstraintAttachment describes the Constrainable structure the availability information is for</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element name="CubeRegion" type="CubeRegionType" minOccurs="1" maxOccurs="1">
						<xs:annotation>
							<xs:documentation>CubeRegion defines a slice of the data set (dimensions and attribute values) for the constrained artefact. A set of included or excluded regions can be described.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
				<xs:attribute name="seriesCount" type="xs:int" use="optional"/>
				<xs:attribute name="obsCount" type="xs:int" use="optional"/>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
