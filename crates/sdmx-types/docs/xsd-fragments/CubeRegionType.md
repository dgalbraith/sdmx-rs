<details>
<summary>XSD contract: <code>CubeRegionType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="CubeRegionType">
		<xs:annotation>
			<xs:documentation>CubeRegionType defines the structure of a data cube region. This is based on the abstract RegionType and simply refines the key and attribute values to conform with what is applicable for dimensions and attributes, respectively. See the documentation of the base type for more details on how a region is defined.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="RegionType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element name="KeyValue" type="CubeRegionKeyType" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element name="Component" type="ComponentValueSetType" minOccurs="0" maxOccurs="unbounded"/>
				</xs:sequence>
				<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="prohibited"/>
				<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="prohibited"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
